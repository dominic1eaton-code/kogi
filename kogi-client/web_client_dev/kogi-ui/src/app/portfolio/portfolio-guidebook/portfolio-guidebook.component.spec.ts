import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioGuidebookComponent } from './portfolio-guidebook.component';

describe('PortfolioGuidebookComponent', () => {
  let component: PortfolioGuidebookComponent;
  let fixture: ComponentFixture<PortfolioGuidebookComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioGuidebookComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioGuidebookComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
