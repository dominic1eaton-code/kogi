import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioCollaborationComponent } from './portfolio-collaboration.component';

describe('PortfolioCollaborationComponent', () => {
  let component: PortfolioCollaborationComponent;
  let fixture: ComponentFixture<PortfolioCollaborationComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioCollaborationComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioCollaborationComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
