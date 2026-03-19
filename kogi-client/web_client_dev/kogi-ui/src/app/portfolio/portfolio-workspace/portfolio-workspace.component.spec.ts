import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioWorkspaceComponent } from './portfolio-workspace.component';

describe('PortfolioWorkspaceComponent', () => {
  let component: PortfolioWorkspaceComponent;
  let fixture: ComponentFixture<PortfolioWorkspaceComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioWorkspaceComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioWorkspaceComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
